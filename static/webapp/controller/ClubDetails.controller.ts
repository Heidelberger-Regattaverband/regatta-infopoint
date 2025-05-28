import Button, { Button$PressEvent } from "sap/m/Button";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import { SearchField$LiveChangeEvent } from "sap/m/SearchField";
import Table from "sap/m/Table";
import { Route$PatternMatchedEvent } from "sap/ui/core/routing/Route";
import Context from "sap/ui/model/Context";
import Filter from "sap/ui/model/Filter";
import FilterOperator from "sap/ui/model/FilterOperator";
import JSONModel from "sap/ui/model/json/JSONModel";
import ListBinding from "sap/ui/model/ListBinding";
import Formatter from "../model/Formatter";
import BaseController from "./Base.controller";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class ClubDetailsController extends BaseController {

  private static readonly CLUB_MODEL: string = "club";
  private static readonly ENTRIES_MODEL: string = "entries";

  readonly formatter: Formatter = Formatter;
  private table?: Table;
  private clubId?: number;

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());

    this.table = super.getView()?.byId("clubEntriesTable") as Table;

    super.setViewModel(new JSONModel(), ClubDetailsController.ENTRIES_MODEL);
    super.setViewModel(new JSONModel(), ClubDetailsController.CLUB_MODEL);

    super.getRouter()?.getRoute("clubDetails")?.attachPatternMatched(
      async (event: Route$PatternMatchedEvent) => await this.onPatternMatched(event), this);
  }

  onNavBack(): void {
    super.navBack("clubs");
    delete this.clubId;
  }

  onSelectionChange(oEvent: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = oEvent.getParameter("listItem");
    if (selectedItem) {
      const bindingCtx: Context | null | undefined = selectedItem.getBindingContext(ClubDetailsController.ENTRIES_MODEL);
      const registration: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());

      registration.race._nav = { disabled: true, back: "clubDetails" };

      (super.getComponentModel("race") as JSONModel).setData(registration.race);
      super.navToRaceDetails(registration.race.id);
    }
  }

  onRefreshButtonPress(event: Button$PressEvent): void {
    const source: Button = event.getSource();
    source.setEnabled(false);
    this.loadData().then((succeeded: [boolean, boolean]) => {
      super.showDataUpdatedMessage(succeeded[0] && succeeded[1]);
    }).finally(() => source.setEnabled(true));
  }

  onSearchFieldLiveChange(event: SearchField$LiveChangeEvent): void {
    const query: string | undefined = event.getParameters().newValue?.trim();
    const searchFilters: Filter[] = query ? this.createSearchFilters(query) : [];

    const binding: ListBinding | undefined = this.table?.getBinding("items") as ListBinding;
    binding?.filter(searchFilters);
  }

  private createSearchFilters(query: string): Filter[] {
    return [new Filter({
      filters: [
        new Filter({
          path: "crew/",
          test: function (crews: any[]) {
            for (const crew of crews) {
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
    await this.loadData();
  }

  private async loadData(): Promise<[boolean, boolean]> {
    const regatta: any = await super.getActiveRegatta();

    const clubUrl: string = `/api/regattas/${regatta.id}/clubs/${this.clubId}`;
    const entriesUrl: string = `/api/regattas/${regatta.id}/clubs/${this.clubId}/entries`;

    const clubModel: JSONModel = super.getViewModel(ClubDetailsController.CLUB_MODEL) as JSONModel;
    const entriesModel: JSONModel = super.getViewModel(ClubDetailsController.ENTRIES_MODEL) as JSONModel;

    return await Promise.all([super.updateJSONModel(entriesModel, entriesUrl, this.table), super.updateJSONModel(clubModel, clubUrl)]);
  }
}