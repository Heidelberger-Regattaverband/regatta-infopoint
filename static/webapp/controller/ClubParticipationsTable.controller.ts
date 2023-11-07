import Table from "sap/m/Table";
import Formatter from "../model/Formatter";
import BaseController from "./Base.controller";
import MyComponent from "de/regatta_hd/Component";
import JSONModel from "sap/ui/model/json/JSONModel";
import MessageToast from "sap/m/MessageToast";
import { ListBase$SelectEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import Button, { Button$PressEvent } from "sap/m/Button";
import Filter from "sap/ui/model/Filter";
import FilterOperator from "sap/ui/model/FilterOperator";
import { SearchField$SearchEvent } from "sap/m/SearchField";
import ListBinding from "sap/ui/model/ListBinding";
import { Route$PatternMatchedEvent } from "sap/ui/core/routing/Route";
import Context from "sap/ui/model/Context";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class ClubParticipationsTable extends BaseController {

  formatter: Formatter = Formatter;
  private table: Table;
  private clubId?: int;

  onInit(): void {
    super.getView()?.addStyleClass((this.getOwnerComponent() as MyComponent).getContentDensityClass());

    this.table = super.getView()?.byId("registrationsTable") as Table;

    super.setViewModel(new JSONModel(), "registrations");
    super.setViewModel(new JSONModel(), "club");

    super.getRouter()?.getRoute("clubParticipations")?.attachPatternMatched(async (event: Route$PatternMatchedEvent) => await this._onPatternMatched(event), this);
  }

  onSelectionChange(oEvent: ListBase$SelectEvent): void {
    const selectedItem: ListItemBase | undefined = oEvent.getParameter("listItem");
    if (selectedItem) {
      const bindingCtx: Context | null | undefined = selectedItem.getBindingContext("registrations");
      const registration: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());

      registration.heat._nav = { disabled: true, back: "clubParticipations" };

      (super.getComponentModel("heat") as JSONModel).setData(registration.heat);
      super.displayTarget("heatRegistrations");
    }
  }

  onNavBack(): void {
    super.navBack("participatingClubs");
    delete this.clubId;
  }

  async onRefreshButtonPress(event: Button$PressEvent): Promise<void> {
    const source: Button = event.getSource();
    source.setEnabled(false);
    await this.loadRegistrationsModel();
    MessageToast.show(this.i18n("msg.dataUpdated", undefined));
    source.setEnabled(true);
  }

  onFilterSearch(event: SearchField$SearchEvent): void {
    const searchFilters: Filter[] = [];
    const query: string | undefined = event.getParameter("query")?.trim();
    if (query) {
      searchFilters.push(
        new Filter({
          filters: [
            new Filter({
              path: "crew/",
              test: function (aCrew) {
                let mCrew;
                for (mCrew of aCrew) {
                  let found = mCrew.athlete.firstName.toLowerCase().includes(query.toLowerCase())
                    || mCrew.athlete.lastName.toLowerCase().includes(query.toLowerCase());
                  if (found) {
                    return true;
                  }
                }
                return false;
              }
            }),
            new Filter("race/number", FilterOperator.Contains, query),
          ],
          and: false
        }))
    }
    const binding: ListBinding = this.table.getBinding("items") as ListBinding;
    binding?.filter(searchFilters);
  }

  private async _onPatternMatched(event: Route$PatternMatchedEvent) {
    this.clubId = (event.getParameter("arguments") as any).clubId;
    await Promise.all([this.loadRegistrationsModel(), this.loadClubModel()]);
  }

  private async loadClubModel(): Promise<void> {
    await super.updateJSONModel(super.getViewModel("club") as JSONModel, `/api/clubs/${this.clubId}`);
  }

  private async loadRegistrationsModel(): Promise<void> {
    await super.updateJSONModel(super.getViewModel("registrations") as JSONModel, `/api/regattas/${super.getRegattaId()}/clubs/${this.clubId}/registrations`, this.table);
  }
}