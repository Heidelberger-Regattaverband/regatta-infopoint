sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/json/JSONModel",
  "sap/m/MessageToast",
  "../model/Formatter"
], function (BaseController, JSONModel, MessageToast, Formatter) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.ClubParticipationsTable", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this._oTable = this.getView().byId("registrationsTable");

      this.setViewModel(new JSONModel(), "registrations");
      this.setViewModel(new JSONModel(), "club");

      this.getRouter().getRoute("clubParticipations").attachPatternMatched(async (oEvent) => await this._onPatternMatched(oEvent), this);
    },

    onSelectionChange: function (oEvent) {
      const oSelectedItem = oEvent.getParameter("listItem");
      if (oSelectedItem) {
        const oBindingCtx = oSelectedItem.getBindingContext("registrations");
        const oRegistration = oBindingCtx.getModel().getProperty(oBindingCtx.getPath());

        oRegistration.heat._nav = { disabled: true, back: "clubParticipations" };

        this.getComponentModel("heat").setData(oRegistration.heat);
        this.displayTarget("heatRegistrations");
      }
    },

    onNavBack: function () {
      this.navBack("participatingClubs");
      delete this._iClubId;
    },

    onRefreshButtonPress: async function (oEvent) {
      const oSource = oEvent.getSource();
      oSource.setEnabled(false);
      await this._loadRegistrationsModel();
      MessageToast.show(this.i18n("msg.dataUpdated", undefined));
      oSource.setEnabled(true);
    },

    _onPatternMatched: async function (oEvent) {
      this._iClubId = oEvent.getParameter("arguments").clubId;

      await Promise.all([this._loadRegistrationsModel(), this._loadClubModel()]);
    },

    _loadClubModel: async function () {
      await this.updateJSONModel(this.getViewModel("club"), `/api/clubs/${this._iClubId}`, undefined);
    },

    _loadRegistrationsModel: async function () {
      await this.updateJSONModel(this.getViewModel("registrations"),
        `/api/regattas/${this.getRegattaId()}/clubs/${this._iClubId}/registrations`, this._oTable);
    }

  });
});