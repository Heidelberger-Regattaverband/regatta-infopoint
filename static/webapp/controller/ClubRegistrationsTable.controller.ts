import Table from "sap/m/Table";
import Formatter from "../model/Formatter";
import BaseController from "./Base.controller";
import JSONModel from "sap/ui/model/json/JSONModel";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import Button, { Button$PressEvent } from "sap/m/Button";
import Filter from "sap/ui/model/Filter";
import FilterOperator from "sap/ui/model/FilterOperator";
import { SearchField$LiveChangeEvent } from "sap/m/SearchField";
import ListBinding from "sap/ui/model/ListBinding";
import { Route$PatternMatchedEvent } from "sap/ui/core/routing/Route";
import Context from "sap/ui/model/Context";
import MessageToast from "sap/m/MessageToast";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class ClubRegistrationsTableController extends BaseController {

  formatter: Formatter = Formatter;
  private table: Table;
  private clubId?: number;
  private readonly registrationsModel: JSONModel = new JSONModel();
  private readonly clubModel: JSONModel = new JSONModel();

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());

    this.table = super.getView()?.byId("registrationsTable") as Table;

    super.setViewModel(this.registrationsModel, "registrations");
    super.setViewModel(this.clubModel, "club");

    super.getRouter()?.getRoute("clubRegistrations")?.attachPatternMatched(
      async (event: Route$PatternMatchedEvent) => await this.onPatternMatched(event), this);
  }

  onNavBack(): void {
    super.navBack("participatingClubs");
    delete this.clubId;
  }

  onSelectionChange(oEvent: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = oEvent.getParameter("listItem");
    if (selectedItem) {
      const bindingCtx: Context | null | undefined = selectedItem.getBindingContext("registrations");
      const registration: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());

      registration.race._nav = { disabled: true, back: "clubRegistrations" };

      (super.getComponentModel("race") as JSONModel).setData(registration.race);
      super.navToRaceDetails(registration.race.id);
    }
  }

  onRefreshButtonPress(event: Button$PressEvent): void {
    const source: Button = event.getSource();
    source.setEnabled(false);
    this.loadRegistrationsModel().then((updated: boolean) => {
      if (updated) {
        MessageToast.show(this.i18n("msg.dataUpdated"));
      }
    }).finally(() => source.setEnabled(true));
  }

  onSearchFieldLiveChange(event: SearchField$LiveChangeEvent): void {
    const query: string | undefined = event.getParameters().newValue?.trim();
    const searchFilters: Filter[] = query ? this.createSearchFilters(query) : [];

    const binding: ListBinding = this.table.getBinding("items") as ListBinding;
    binding.filter(searchFilters);
  }

  private createSearchFilters(query: string): Filter[] {
    return [new Filter({
      filters: [
        new Filter({
          path: "crew/",
          test: function (crews: any[]) {
            for (let crew of crews) {
              const found = crew.athlete.firstName.toLowerCase().includes(query.toLowerCase())
                || crew.athlete.lastName.toLowerCase().includes(query.toLowerCase());
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
    })]
  }

  private async onPatternMatched(event: Route$PatternMatchedEvent): Promise<void> {
    this.clubId = (event.getParameter("arguments") as any).clubId;
    await Promise.all([this.loadRegistrationsModel(), this.loadClubModel()]);
  }

  private async loadClubModel(): Promise<boolean> {
    return await super.updateJSONModel(this.clubModel, `/api/regattas/${super.getRegattaId()}/clubs/${this.clubId}`);
  }

  private async loadRegistrationsModel(): Promise<boolean> {
    return await super.updateJSONModel(this.registrationsModel,
      `/api/regattas/${super.getRegattaId()}/clubs/${this.clubId}/registrations`, this.table);
  }
}